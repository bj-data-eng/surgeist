//! Authored tracking at actual TCU boundaries, with no spacing at line edges.
//! CSS Text4 §8.2: adjacent half contributions sum to their average.
use super::*;
use crate::{Id, LineHeight, Range, test_fonts as sfnt};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn own(letter_spacing: f64) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        letter_spacing,
        ..Style::default()
    }
}

fn font_premises(bytes: &[u8], marked: bool) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.maxp().unwrap().num_glyphs(), 281);
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map('B'), Some(GlyphId::new(36)));
    assert!(font.table_data(skrifa::Tag::new(b"fvar")).is_none());
    for tag in [*b"glyf", *b"loca", *b"hmtx", *b"hhea"] {
        assert_eq!(sfnt::table(bytes, tag), sfnt::table(AHEM, tag));
    }
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for id in [35, 36] {
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
        let bounds = metrics.bounds(GlyphId::new(id)).unwrap();
        assert_eq!(
            [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
            [0.0, -200.0, 1000.0, 800.0]
        );
    }
    if marked {
        assert_eq!(font.charmap().map('\u{301}'), Some(GlyphId::new(100)));
        assert!(font.gpos().unwrap().lookup_list().is_ok());
        // Read the actual fixture's lookup and anchor offsets independently of
        // the provider. Mark100 attaches only to base35, at one mark class.
        let gpos = sfnt::table(bytes, *b"GPOS");
        let list = usize::from(word(gpos, 8));
        assert_eq!(word(gpos, list), 1);
        let lookup = list + usize::from(word(gpos, list + 2));
        assert_eq!(word(gpos, lookup), 4);
        assert_eq!(word(gpos, lookup + 4), 1);
        let sub = lookup + usize::from(word(gpos, lookup + 6));
        assert_eq!(word(gpos, sub), 1);
        assert_eq!(word(gpos, sub + 6), 1);
        for (offset, glyph) in [(2, 100), (4, 35)] {
            let coverage = sub + usize::from(word(gpos, sub + offset));
            assert_eq!(word(gpos, coverage), 1);
            assert_eq!(word(gpos, coverage + 2), 1);
            assert_eq!(word(gpos, coverage + 4), glyph);
        }
        let marks = sub + usize::from(word(gpos, sub + 8));
        assert_eq!(word(gpos, marks), 1);
        assert_eq!(word(gpos, marks + 2), 0);
        let mark = marks + usize::from(word(gpos, marks + 4));
        let bases = sub + usize::from(word(gpos, sub + 10));
        assert_eq!(word(gpos, bases), 1);
        let base = bases + usize::from(word(gpos, bases + 2));
        assert_eq!((word(gpos, mark), word(gpos, base)), (1, 1));
        assert_eq!((word(gpos, mark + 2), word(gpos, mark + 4)), (100, 200));
        assert_eq!((word(gpos, base + 2), word(gpos, base + 4)), (400, 600));
    }
}

fn word(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn prepare(bytes: &[u8], source: &Source, style: Style, graphemes: &[Range]) -> PreparedText {
    let extent = Range::new(0, source.text().len());
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: style,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: extent,
                    aggregate: extent,
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analyzed.graphemes().collect::<Vec<_>>(), graphemes);
    assert_eq!(analyzed.source(), source);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), source);
    assert_eq!(prepared.orientation(), LineOrientation::Horizontal);
    assert_eq!(
        prepared.terminal_edge(),
        crate::ParagraphTerminalEdge::BlockEnd
    );
    let origins = prepared.origin_views().collect::<Vec<_>>();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0].source(), source);
    assert_eq!(origins[0].local_range(), extent);
    assert_eq!(origins[0].aggregate_range(), extent);
    prepared
}

fn whole(prepared: &PreparedText) -> FinalizedLine {
    let mut request = request(prepared);
    request.context.available_inline = None;
    request.context.spacing = SpacingPolicy::None;
    let selected = prepared.select(request).unwrap();
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(line.selected_line(), selected);
    assert_eq!(line.prepared_text(), *prepared);
    line
}

type RawGlyph = (Range, u32, f32, f32, f32);

fn raw(line: &FinalizedLine, bytes: &[u8]) -> Vec<RawGlyph> {
    let mut result = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        result.extend(run.glyphs().map(|glyph| {
            (
                glyph.range(),
                glyph.id(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            )
        }));
    }
    result
}

/// Glyph paint offsets in the logical inline / baseline-relative under frame.
fn paint_offsets(line: &FinalizedLine) -> Vec<(f64, f64)> {
    line.run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .map(move |glyph| {
                    (
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y - line.feedback().baseline + f64::from(glyph.y()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn equal_tracking_adds_one_internal_gap_and_no_outer_line_spacing() {
    font_premises(AHEM, false);
    let source = Source::identified("AA", Id::from_u64(70001), 1);
    let graphemes = [Range::new(0, 1), Range::new(1, 2)];
    let control = whole(&prepare(AHEM, &source, own(0.0), &graphemes));
    let expected_raw = [
        (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 35, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&control, AHEM), expected_raw);
    assert_eq!(control.feedback().advance, 20.0);
    assert_eq!(paint_offsets(&control), [(0.0, 0.0), (10.0, 0.0)]);
    let prepared = prepare(AHEM, &source, own(4.0), &graphemes);
    let line = whole(&prepared);
    assert_eq!(raw(&line, AHEM), expected_raw);
    // Half2 from each adjoining TCU makes one4px gap. Neither outer half
    // survives, so the two10px glyphs occupy an attributed24px line.
    assert_eq!(line.feedback().advance, 24.0);
    assert_eq!(line.measurements().alignment().advance(), 24.0);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (14.0, 0.0)]);
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn different_adjacent_tracking_styles_contribute_their_average_internal_gap() {
    font_premises(AHEM, false);
    let control_source = Source::new("AB");
    let graphemes = [Range::new(0, 1), Range::new(1, 2)];
    let control = whole(&prepare(AHEM, &control_source, own(0.0), &graphemes));
    let expected_raw = [
        (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 36, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&control, AHEM), expected_raw);
    assert_eq!(control.feedback().advance, 20.0);
    assert_eq!(paint_offsets(&control), [(0.0, 0.0), (10.0, 0.0)]);
    let mut source = Source::identified("AB", Id::from_u64(70002), 1);
    source.span(Range::new(1, 2), own(8.0));
    assert_eq!(source.spans()[0].range(), Range::new(1, 2));
    assert_eq!(source.spans()[0].style().letter_spacing, 8.0);
    let prepared = prepare(AHEM, &source, own(4.0), &graphemes);
    let line = whole(&prepared);
    assert_eq!(raw(&line, AHEM), expected_raw);
    // A contributes2 on its adjoining side, B contributes4: gap6, not4/8.
    assert_eq!(line.feedback().advance, 26.0);
    assert_eq!(line.measurements().alignment().advance(), 26.0);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (16.0, 0.0)]);
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn base_and_attached_mark_share_one_tracking_unit_and_keep_the_actual_gpos_anchor() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    font_premises(&bytes, true);
    let source = Source::identified("A\u{301}B", Id::from_u64(70003), 1);
    let graphemes = [Range::new(0, 2), Range::new(2, 3)];
    let control = whole(&prepare(&bytes, &source, own(0.0), &graphemes));
    let expected_raw = [
        (Range::new(0, 2), 35, 0.0, 0.0, 10.0),
        (Range::new(0, 2), 100, -7.0, -4.0, 0.0),
        (Range::new(2, 3), 36, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&control, &bytes), expected_raw);
    // Actual base(400,600)−mark(100,200), scaled by10/1000 and with
    // font-positive-up converted to under-positive-down, is(+3,−4).
    assert_eq!(
        paint_offsets(&control),
        [(0.0, 0.0), (3.0, -4.0), (10.0, 0.0)]
    );
    assert_eq!(control.feedback().advance, 20.0);
    let prepared = prepare(&bytes, &source, own(4.0), &graphemes);
    let line = whole(&prepared);
    assert_eq!(raw(&line, &bytes), expected_raw);
    assert_eq!(line.feedback().advance, 24.0);
    assert_eq!(line.measurements().alignment().advance(), 24.0);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (3.0, -4.0), (14.0, 0.0)]);
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn mark_only_tracking_style_preserves_joint_gpos_and_uses_the_tcu_base_tracking() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    font_premises(&bytes, true);
    let graphemes = [Range::new(0, 2), Range::new(2, 3)];
    let mut control_source = Source::identified("A\u{301}B", Id::from_u64(70004), 1);
    control_source.span(Range::new(1, 2), own(0.0));
    assert_eq!(control_source.spans()[0].range(), Range::new(1, 2));
    assert_eq!(control_source.spans()[0].style().letter_spacing, 0.0);
    let control = whole(&prepare(&bytes, &control_source, own(0.0), &graphemes));
    let expected_raw = [
        (Range::new(0, 2), 35, 0.0, 0.0, 10.0),
        (Range::new(0, 2), 100, -7.0, -4.0, 0.0),
        (Range::new(2, 3), 36, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&control, &bytes), expected_raw);
    assert_eq!(
        paint_offsets(&control),
        [(0.0, 0.0), (3.0, -4.0), (10.0, 0.0)]
    );
    assert_eq!(control.feedback().advance, 20.0);

    let base_style = own(4.0);
    let mut mark_style = base_style.clone();
    // Only tracking differs: this span does not change the actual font, size,
    // feature settings, locale or other shaping inputs of the attached mark.
    mark_style.letter_spacing = 8.0;
    let mut source = Source::identified("A\u{301}B", Id::from_u64(70004), 2);
    source.span(Range::new(1, 2), mark_style);
    assert_eq!(source.spans()[0].range(), control_source.spans()[0].range());
    assert_eq!(source.spans()[0].style().letter_spacing, 8.0);
    let prepared = prepare(&bytes, &source, base_style, &graphemes);
    assert_eq!(prepared.analyzed_source().style().letter_spacing, 4.0);
    let line = whole(&prepared);
    // GPOS still shapes the complete A+mark TCU. The mark's own tracking
    // value cannot separate that invocation or move its(+3,−4) attachment.
    assert_eq!(raw(&line, &bytes), expected_raw);
    assert_eq!(line.feedback().advance, 24.0);
    assert_eq!(line.measurements().alignment().advance(), 24.0);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (3.0, -4.0), (14.0, 0.0)]);
    assert_eq!(prepared.analyzed_source().source(), &source);
}
