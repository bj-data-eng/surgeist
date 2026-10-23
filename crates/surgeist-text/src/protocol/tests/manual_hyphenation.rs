//! Bare authored SHY under the initial manual/unknown-convention policy.
use super::*;
use crate::{PlaintextItem, PreparedAlternativeDefinition, Range};
use skrifa::MetadataProvider;

fn fixture(primary_hyphen: bool) -> Vec<u8> {
    // Actual cmap over the licensed Ahem outlines/metrics. The optional AA
    // fixture lookup cannot match AX, A+HYPHEN, or A+HYPHEN-MINUS here.
    let mut mappings = vec![(0x20, 3), (0x2d, 15), (0x41, 35), (0x58, 58), (0xad, 3)];
    if primary_hyphen {
        mappings.push((0x2010, 36));
    }
    crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &mappings)
}

fn prepare(bytes: &[u8]) -> PreparedText {
    let source = Source::new("A\u{ad}X");
    let value = ParagraphInput {
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            locale: None,
            ..style()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            source,
            local: Range::new(0, 3),
            aggregate: Range::new(0, 3),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        // A real bare SHY: the caller supplies no resolved replacement string.
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value).unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}
fn plaintext(prepared: &PreparedText) -> Vec<u16> {
    let mut result = Vec::new();
    for item in prepared
        .analyzed_source()
        .processed_source()
        .plaintext_items(crate::Affinity::Before)
    {
        match item {
            PlaintextItem::Text(text) => result.extend_from_slice(text.code_units()),
            PlaintextItem::InlineBox(_) => panic!("fixture has no box occurrences"),
        }
    }
    result
}
fn glyphs(line: &FinalizedLine, bytes: &[u8]) -> Vec<(Range, u32, f32)> {
    let mut result = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 20.0);
        result.extend(
            run.glyphs()
                .filter(|glyph| glyph.advance() != 0.0)
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    result.sort_by_key(|(range, _, _)| (range.start, range.end));
    result
}

fn bare_shy(primary_hyphen: bool) {
    let bytes = fixture(primary_hyphen);
    // Independently inspect the actual selected primary font's nominal coverage;
    // neither indication choice nor expected glyph comes from engine output.
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let cmap = font.charmap();
    assert_eq!(cmap.map(0x41_u32).map(|id| id.to_u32()), Some(35));
    assert_eq!(cmap.map(0x58_u32).map(|id| id.to_u32()), Some(58));
    assert_eq!(cmap.map(0x2d_u32).map(|id| id.to_u32()), Some(15));
    assert_eq!(
        cmap.map(0x2010_u32).map(|id| id.to_u32()),
        primary_hyphen.then_some(36)
    );
    let (indication, glyph) = if primary_hyphen {
        (0x2010, 36)
    } else {
        (0x2d, 15)
    };
    let prepared = prepare(&bytes);
    let original = [65, 0xad, 88];
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        original
    );
    assert_eq!(plaintext(&prepared), original);

    // A natural paragraph end never takes the conditional indication, including
    // when a later selection will genuinely use this same source SHY.
    let whole = prepared
        .select(request(&prepared))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(whole.selected_line().request().alternative.is_none());
    assert_eq!(whole.feedback().advance, 40.0);
    assert_eq!(
        glyphs(&whole, &bytes),
        [(Range::new(0, 1), 35, 20.0), (Range::new(2, 3), 58, 20.0)]
    );
    assert!(
        !whole
            .mapping_views()
            .any(|piece| piece.kind() == PresentationMappingKind::Replacement)
    );

    // Initial hyphens:manual needs no language dictionary; bare SHY must issue
    // a real used indication rather than masquerading as a generic soft break.
    assert_eq!(
        prepared
            .opportunity_views()
            .map(|view| (view.boundary().view().source_before().get(), view.kind()))
            .collect::<Vec<_>>(),
        [
            (2, PreparedBreakKind::Discretionary),
            (3, PreparedBreakKind::ParagraphEnd)
        ]
    );
    let offer = prepared
        .opportunity_views()
        .find(|view| view.kind() == PreparedBreakKind::Discretionary)
        .unwrap();
    let alternatives = offer.alternatives().collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 1);
    let alternative = &alternatives[0];
    assert_eq!(alternative.text().code_units(), [indication]);
    assert_eq!(alternative.source_range(), Some(Range::new(1, 2)));
    assert_eq!(alternative.insertion_anchor().get(), 2);
    assert_eq!(
        alternative.definition(),
        PreparedAlternativeDefinition::Source {
            origin: prepared.origin_views().next().unwrap().handle(),
            source: Range::new(1, 2)
        }
    );
    assert_eq!(alternative.opportunity(), offer.handle());
    let mut value = request(&prepared);
    value.end = offer.boundary();
    value.participants = prepared
        .participants()
        .take(value.end.view().participant_offset())
        .collect();
    value.termination = PreparedTermination::Ordinary(offer.handle());
    value.alternative = Some(alternative.handle());
    value.context.end = LineEnd::Soft;
    let first = prepared
        .select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(first.feedback().advance, 40.0);
    assert_eq!(
        glyphs(&first, &bytes),
        [
            (Range::new(0, 1), 35, 20.0),
            (Range::new(2, 2), glyph, 20.0)
        ]
    );
    assert!(
        first
            .mapping_views()
            .any(|piece| piece.kind() == PresentationMappingKind::Replacement
                && piece.alternative() == Some(alternative.handle()))
    );
    let mut value = request(&prepared);
    value.start = first.selected_line().request().end.clone();
    value.participants = prepared
        .participants()
        .skip(value.start.view().participant_offset())
        .collect();
    value.context.start = LineStart::Continuation;
    let second = prepared
        .select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(second.feedback().advance, 20.0);
    assert!(second.selected_line().request().alternative.is_none());
    assert_eq!(glyphs(&second, &bytes), [(Range::new(2, 3), 58, 20.0)]);
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        lines: [first, second]
            .into_iter()
            .enumerate()
            .map(|(index, line)| LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0 + index as f64 * 20.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.source().text().code_units(), original);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let selection = crate::Selection::new(
        crate::Cursor::new(0, crate::Affinity::After),
        crate::Cursor::new(3, crate::Affinity::Before),
    );
    assert_eq!(selection.range(), Range::new(0, 3));
    assert_eq!(layout.selection(selection).unwrap().rects().len(), 2);
    assert_eq!(plaintext(&prepared), original);
}
#[test]
fn bare_manual_shy_uses_primary_u2010_without_a_caller_replacement() {
    bare_shy(true);
}
#[test]
fn bare_manual_shy_falls_back_to_u002d_when_primary_u2010_is_absent() {
    bare_shy(false);
}

fn raw_glyphs(layout: &crate::Layout, bytes: &[u8]) -> Vec<(usize, Range, u32, f32)> {
    let mut rows = Vec::new();
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            layout.key().font_catalogue()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 20.0);
    }
    for run in layout.glyph_runs() {
        rows.extend(
            run.glyphs()
                .iter()
                .filter(|glyph| glyph.advance() != 0.0)
                .map(|glyph| (run.line(), glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    rows.sort_by_key(|(line, range, _, _)| (*line, range.start, range.end));
    rows
}
fn raw_line_advances(layout: &crate::Layout) -> Vec<f64> {
    let mut advances = vec![0.0; layout.lines().len()];
    // These published finalized run facts include the actual used insertion.
    // This fixture has no inline boxes or extra spacing allocations.
    for run in layout.inline_metric_facts().runs() {
        advances[run.line()] += run.advance();
    }
    advances
}
fn raw_bare_shy(primary_hyphen: bool) {
    let bytes = fixture(primary_hyphen);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let cmap = font.charmap();
    assert_eq!(cmap.map(0x58_u32).map(|glyph| glyph.to_u32()), Some(58));
    assert_eq!(cmap.map(0x2d_u32).map(|glyph| glyph.to_u32()), Some(15));
    assert_eq!(
        cmap.map(0x2010_u32).map(|glyph| glyph.to_u32()),
        primary_hyphen.then_some(36)
    );
    let hyphen = if primary_hyphen { 36 } else { 15 };
    // Three X glyphs, no AA components: the fixture's optional ligature lookup
    // cannot alter the independent 20px contributions or the fit threshold.
    let source = Source::new("X\u{ad}XX");
    let original = [88, 0xad, 88, 88];
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let unbroken = system
        .layout(
            source.clone(),
            style(),
            crate::Options {
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    let expected_unbroken = vec![
        (0, Range::new(0, 1), 58, 20.0),
        (0, Range::new(2, 3), 58, 20.0),
        (0, Range::new(3, 4), 58, 20.0),
    ];
    assert_eq!(unbroken.source().text().code_units(), original);
    assert_eq!(raw_glyphs(&unbroken, &bytes), expected_unbroken);
    assert_eq!(raw_line_advances(&unbroken), [60.0]);
    assert_eq!(
        unbroken
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 4)]
    );

    let wrapped = system
        .layout(
            source.clone(),
            style(),
            crate::Options {
                width: Some(40.0),
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    drop(system);
    // The subsequent narrow request cannot mutate the earlier font/line result.
    assert_eq!(raw_glyphs(&unbroken, &bytes), expected_unbroken);
    assert_eq!(unbroken.metrics().full_width(), 60.0);
    assert_eq!(wrapped.source(), &source);
    assert_eq!(wrapped.source().text().code_units(), original);
    let processed = crate::process_source_whitespace(
        wrapped.source().clone(),
        style(),
        crate::DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    let mut plain = Vec::new();
    for item in processed.plaintext_items(crate::Affinity::Before) {
        match item {
            PlaintextItem::Text(text) => plain.extend_from_slice(text.code_units()),
            PlaintextItem::InlineBox(_) => panic!("raw fixture has no boxes"),
        }
    }
    assert_eq!(plain, original);
    let actual = (
        wrapped
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        raw_line_advances(&wrapped),
        raw_glyphs(&wrapped, &bytes),
    );
    // A taken initial manual SHY contributes its actual used indication. The
    // first line is X20+hyphen20, and the continuation contains the other XX40.
    // This qualifier uses only the unchanged public raw System::layout front.
    assert_eq!(
        actual,
        (
            vec![Range::new(0, 2), Range::new(2, 4)],
            vec![40.0, 40.0],
            vec![
                (0, Range::new(0, 1), 58, 20.0),
                (0, Range::new(2, 2), hyphen, 20.0),
                (1, Range::new(2, 3), 58, 20.0),
                (1, Range::new(3, 4), 58, 20.0)
            ],
        )
    );
    assert_eq!(wrapped.metrics().full_width(), 40.0);
    assert!(!wrapped.metrics().overflow());
    let selection = crate::Selection::new(
        crate::Cursor::new(0, crate::Affinity::After),
        crate::Cursor::new(4, crate::Affinity::Before),
    );
    assert_eq!(wrapped.selection(selection).unwrap().rects().len(), 2);
    assert_eq!(wrapped.source().text().code_units(), original);
}
#[test]
fn raw_manual_shy_takes_primary_u2010_in_the_real_line_budget() {
    raw_bare_shy(true);
}
#[test]
fn raw_manual_shy_takes_fallback_u002d_in_the_real_line_budget() {
    raw_bare_shy(false);
}
