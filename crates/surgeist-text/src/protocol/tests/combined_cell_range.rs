//! Initial existing-callable Cell Range proof; private child of
//! combined_cell_observation_boundaries. No future geometry enum is referenced.
//!
//! Its prepared_cell("AA") helper issues two separate one-character origins,
//! which cannot authorize a single original-local0..2 request. These tests
//! instead reuse the existing widths single-origin factory with ALL three width
//! tags genuinely authored off before Source capture. This is its actual unique
//! combined-width Ahem derivative, not the observation helper's NoAction font.
//! Its normal AA10/10, disabled-action receipts, H8/-2 metrics and .5 frame are
//! qualified independently below. Placement reuses the observation helper.

use super::*;
use surgeist_shaper::{
    Direction as ProviderDirection, Feature, FontRef as ProviderFont, ShaperData, ShapingStatus,
    Tag, UnicodeBuffer, WidthFeature, WidthInputOutcome, WidthObservationCoverage, script,
};

const OFF: [&str; 3] = ["\"hwid\" off", "\"twid\" off", "\"qwid\" off"];
const FIRST: Rect = Rect::new(95.0, 20.0, 5.0, 10.0);
const SECOND: Rect = Rect::new(100.0, 20.0, 5.0, 10.0);

fn provider_premises(bytes: &[u8]) {
    glyph_frame_font_premises(bytes);
    let font = ProviderFont::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for width in [
        None,
        Some((*b"hwid", WidthFeature::HalfWidth)),
        Some((*b"twid", WidthFeature::ThirdWidth)),
        Some((*b"qwid", WidthFeature::QuarterWidth)),
    ] {
        let mut input = UnicodeBuffer::new();
        input.add('A', 0);
        input.add('A', 1);
        input.set_direction(ProviderDirection::LeftToRight);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        // Match the real candidate precedence: a trial request precedes the
        // authored settings. Each last authored zero disables its own tag.
        let mut features = Vec::new();
        if let Some((tag, feature)) = width {
            input.observe_width_feature(feature);
            features.push(Feature::new(Tag::new(&tag), 1, ..));
        }
        for tag in [*b"hwid", *b"twid", *b"qwid"] {
            features.push(Feature::new(Tag::new(&tag), 0, ..));
        }
        let output = shaper.shape(input, &features);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(35, 0), (35, 1)]
        );
        assert_eq!(
            output
                .glyph_positions()
                .iter()
                .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
                .collect::<Vec<_>>(),
            [(1000, 0, 0, 0); 2]
        );
        if let Some((_, feature)) = width {
            let receipt = output.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.feature(), feature);
            assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
            assert_eq!(receipt.input_len(), 2);
            for index in 0..2 {
                assert_eq!(
                    receipt.input(index).unwrap().outcome(),
                    WidthInputOutcome::NoAction
                );
            }
        } else {
            assert!(output.width_feature_receipt().unwrap().is_none());
        }
    }
}

fn rectangles(result: &SourceRangeFragments<'_>) -> Vec<Rect> {
    result
        .units()
        .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
        .collect()
}

fn exact(
    result: &SourceRangeFragments<'_>,
    origin: &PreparedOrigin,
    query: Range,
    expected: &[(Range, Rect)],
) {
    assert_eq!(result.origin(), origin);
    assert_eq!(result.local_range(), query);
    let units = result.units().collect::<Vec<_>>();
    assert_eq!(units.len(), expected.len());
    assert_eq!(
        rectangles(result),
        expected.iter().map(|(_, rect)| *rect).collect::<Vec<_>>()
    );
    for (unit, (source, rect)) in units.iter().zip(expected) {
        assert_eq!(unit.line_index(), 0);
        assert_eq!(unit.pieces().count(), 1);
        assert_eq!(unit.pieces().next().unwrap().rect(), *rect);
        for intersections in [
            unit.selected_intersections().collect::<Vec<_>>(),
            unit.expanded_intersections().collect::<Vec<_>>(),
        ] {
            assert_eq!(intersections.len(), 1);
            assert_eq!(intersections[0].origin(), *origin);
            assert_eq!(intersections[0].local_range(), *source);
            assert_eq!(intersections[0].aggregate_range(), *source);
        }
    }
}

fn ordinary_control(bytes: &[u8]) {
    let source = Source::identified("AA", Id::from_u64(784_993_401), 1);
    let extent = Range::new(0, 2);
    let mut font = Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY);
    for setting in OFF {
        font = font.feature(setting);
    }
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font,
                size: 10.0,
                line_height: LineHeight::Absolute(10.0),
                white_space: WhiteSpace::Preserve,
                text_combine_upright: TextCombineUpright::None,
                wrap: Wrap::Word,
                ..Style::default()
            },
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            origins: vec![ParagraphOrigin {
                source: source.clone(),
                text_node: None,
                local: extent,
                aggregate: extent,
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.combined_cells().len(), 0);
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    assert_eq!(origin.view().local_range(), extent);
    let line = complete_line(&prepared);
    assert_eq!(line.feedback().advance, 20.0);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 95.0,
                baseline: 28.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    // Same real font metrics8/-2, uncompressed horizontal advances10/10.
    // The ordinary observer must stay available before Cell support is asked.
    for (query, expected) in [
        (
            Range::new(0, 1),
            vec![(Range::new(0, 1), Rect::new(95.0, 20.0, 10.0, 10.0))],
        ),
        (
            Range::new(1, 2),
            vec![(Range::new(1, 2), Rect::new(105.0, 20.0, 10.0, 10.0))],
        ),
        (
            extent,
            vec![
                (Range::new(0, 1), Rect::new(95.0, 20.0, 10.0, 10.0)),
                (Range::new(1, 2), Rect::new(105.0, 20.0, 10.0, 10.0)),
            ],
        ),
    ] {
        exact(
            &layout.source_range_fragments(&origin, query).unwrap(),
            &origin,
            query,
            &expected,
        );
    }
}

pub(super) fn fixture() -> (Layout, PreparedOrigin) {
    let (mut system, processed, analyzed, bytes) = source_with_font_features("AA", &OFF);
    provider_premises(&bytes);
    ordinary_control(&bytes);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let origins = prepared.origin_views().collect::<Vec<_>>();
    let [view] = origins.as_slice() else {
        panic!("the genuine single-origin factory must retain one full origin");
    };
    assert_eq!(view.source(), processed.source());
    assert_eq!(view.local_range(), Range::new(0, 2));
    assert_eq!(view.aggregate_range(), Range::new(0, 2));
    assert_eq!(
        view.capture_position(),
        Some(processed.captured_origin(0).unwrap())
    );
    let origin = view.handle();
    let line = complete_line(&prepared);
    one_em(&line);
    let raw = line
        .run_views()
        .flat_map(|run| {
            assert!(run.cell().is_some());
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            let instance = run.effective_instance().unwrap();
            let metrics = instance
                .metrics(instance.face().catalogue_identity())
                .unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        g.id(),
                        run.origin().x + g.pen(),
                        g.advance(),
                        g.x(),
                        g.y(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw,
        [
            (Range::new(0, 1), 35, 0.0, 10.0, 0.0, 0.0),
            (Range::new(1, 2), 35, 10.0, 10.0, 0.0, 0.0)
        ]
    );
    let layout = placed_cell(&prepared, line, &bytes, 0.5);
    assert_eq!(layout.source(), processed.source());
    let runs = layout.glyph_runs();
    let [run] = runs.as_slice() else {
        panic!("one real retained normal-AA packet");
    };
    assert_eq!(run.font().bytes(), bytes.as_slice());
    let frame = run.glyph_frame();
    assert_eq!(frame.origin(), Point::new(95.0, 28.0));
    assert_eq!(frame.outline_x(), Point::new(0.5, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    // Independent metric corners: raw horizontal edges0/10/20, font vertical
    // edges-8/+2, frame(95,28), scale.5 => [95,100]/[100,105] x [20,30].
    // Existing placed cluster geometry is a control, never the Range oracle.
    let mut clusters = layout
        .clusters()
        .into_iter()
        .map(|row| (row.range(), row.bounds()))
        .collect::<Vec<_>>();
    clusters.sort_by_key(|(range, _)| range.start);
    assert_eq!(
        clusters,
        [(Range::new(0, 1), FIRST), (Range::new(1, 2), SECOND)]
    );
    drop(system);
    (layout, origin)
}

#[test]
fn first_original_a_range_uses_only_its_actual_inner_metric_rectangle() {
    let (layout, origin) = fixture();
    let query = Range::new(0, 1);
    let result = layout.source_range_fragments(&origin, query).expect(
        "the first real Cell EGC must expose its compressed physical font-metric rectangle",
    );
    exact(&result, &origin, query, &[(query, FIRST)]);
}

#[test]
fn second_original_a_range_uses_the_second_actual_inner_metric_rectangle() {
    let (layout, origin) = fixture();
    let query = Range::new(1, 2);
    let result = layout
        .source_range_fragments(&origin, query)
        .expect("the second real Cell EGC must not reuse the square or first EGC rectangle");
    exact(&result, &origin, query, &[(query, SECOND)]);
}

#[test]
fn full_original_aa_range_retains_two_distinct_inner_units_and_complete_source_coverage() {
    let (layout, origin) = fixture();
    let query = Range::new(0, 2);
    let full = layout.source_range_fragments(&origin, query).expect(
        "a full Cell Range must retain both genuine semantic EGCs rather than one square unit",
    );
    exact(
        &full,
        &origin,
        query,
        &[(Range::new(0, 1), FIRST), (Range::new(1, 2), SECOND)],
    );
    let first = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    let second = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    let units = full.units().collect::<Vec<_>>();
    assert!(!units[0].same_unit(&units[1]));
    assert!(units[0].same_unit(&first.units().next().unwrap()));
    assert!(units[1].same_unit(&second.units().next().unwrap()));
    assert_eq!(rectangles(&full), [FIRST, SECOND]);
}

#[path = "combined_cell_range_geometry.rs"]
mod geometry;

#[path = "combined_cell_range_baseline.rs"]
mod baseline;

#[path = "combined_cell_range_source.rs"]
mod source;
