//! Native upright scope receipts and source geometry from real SFNT data.
//! Functional qualification of the captured API, without browser-parity claims.
use crate::*;
use crate::{baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};
use std::sync::Arc;

fn reference(iso: [u8; 4]) -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(iso).unwrap(),
        axis: FontBaselineAxis::VerticalX,
        baseline: FontBaseline::Central,
    }
}
fn native_font() -> Vec<u8> {
    let horizontal = vertical::horizontal(800, -200, 0, 1001);
    let count = sfnt::u16_at(sfnt::table(&horizontal, *b"maxp"), 4);
    let native = vertical::with_native(&horizontal, 1200, 100, count);
    let axis = base::Axis {
        tags: vec![*b"ideo", *b"idtp"],
        scripts: vec![
            base::Script {
                tag: *b"hani",
                default_index: 0,
                coordinates: vec![base::Coord::Design(0), base::Coord::Design(1600)],
            },
            base::Script {
                tag: *b"latn",
                default_index: 0,
                coordinates: vec![base::Coord::Design(0), base::Coord::Design(1200)],
            },
        ],
    };
    sfnt::replace_tables(
        &native,
        vec![(*b"BASE", base::base(0, None, Some(axis), None))],
    )
}
fn premises(bytes: &[u8]) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.charmap().map('X'), Some(GlyphId::new(58)));
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(58)), Some(1001.0));
    let bounds = metrics.bounds(GlyphId::new(58)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
    assert_eq!(font.vhea().unwrap().version().to_major_minor(), (1, 0));
    let vmtx = font.vmtx().unwrap();
    assert_eq!(
        (
            vmtx.v_metrics()[58].advance.get(),
            vmtx.v_metrics()[58].side_bearing.get()
        ),
        (1200, 100)
    );
    let table = font.base().unwrap();
    assert!(table.horiz_axis().is_none());
    let axis = table.vert_axis().unwrap().unwrap();
    assert_eq!(
        axis.base_tag_list()
            .unwrap()
            .unwrap()
            .baseline_tags()
            .iter()
            .map(|tag| tag.get().to_be_bytes())
            .collect::<Vec<_>>(),
        [*b"ideo", *b"idtp"]
    );
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(scripts.base_script_records().len(), 2);
    for (record, (tag, edges)) in scripts
        .base_script_records()
        .iter()
        .zip([(*b"hani", [0, 1600]), (*b"latn", [0, 1200])])
    {
        assert_eq!(record.base_script_tag().to_be_bytes(), tag);
        let values = record
            .base_script(scripts.offset_data())
            .unwrap()
            .base_values()
            .unwrap()
            .unwrap();
        assert_eq!(
            values
                .base_coords()
                .iter()
                .map(|v| v.unwrap().coordinate())
                .collect::<Vec<_>>(),
            edges
        );
    }
}
fn input(text: &str, outside: bool) -> ContextualSourceInput {
    let mut value = if outside {
        super::prepared::two_leaves()
    } else {
        super::capture::captured_input(1)
    };
    let source = Source::identified(text, Id::from_u64(1578070), 41);
    value.source = source.clone();
    let ranges = if outside {
        vec![Range::new(0, 1), Range::new(1, 2)]
    } else {
        vec![Range::new(0, text.encode_utf16().count())]
    };
    value.origins = ranges
        .into_iter()
        .map(|range| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    value.scope_alignment_limits = Some(super::capture::generous());
    value.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
    let SourceProcessingScope::Block {
        base_style,
        font_reference,
        ..
    } = &mut value.scopes[0]
    else {
        unreachable!()
    };
    *base_style = super::capture::style(
        if outside { 10.0 } else { 20.0 },
        if outside { 12.0 } else { 32.0 },
    );
    *font_reference = Some(reference(*b"Hani"));
    let SourceProcessingScope::Inline {
        style,
        font_reference,
        alignment_boundary,
        ..
    } = &mut value.scopes[1]
    else {
        unreachable!()
    };
    *style = Some(super::capture::style(10.0, 12.0));
    *font_reference = Some(reference(*b"Latn"));
    *alignment_boundary = Some(ScopeAlignmentBoundary::NonInitial);
    author_child_body(
        &mut value,
        if outside {
            Range::new(1, 2)
        } else {
            Range::new(0, text.encode_utf16().count())
        },
    );
    value
}
fn author_child_body(value: &mut ContextualSourceInput, range: Range) {
    let SourceProcessingScope::Inline {
        style: Some(style), ..
    } = &value.scopes[1]
    else {
        unreachable!()
    };
    // Captured font context does not restyle text. Author this body's style
    // explicitly on the source while retaining the distinct parent context.
    value.source.span(range, style.clone());
    for origin in &mut value.origins {
        origin.source = value.source.clone();
    }
}
fn analyzed(
    input: ContextualSourceInput,
    typography: AnalysisTypography,
) -> (Arc<ProcessedSource>, AnalyzedSource) {
    let captured = Arc::new(process_contextual_source(input).unwrap());
    let analyzed =
        AnalyzedSource::from_processed_paragraph(captured.paragraphs().next().unwrap(), typography)
            .unwrap();
    (
        captured,
        crate::test_wrapping::bind_missing(&analyzed, Wrap::Word),
    )
}
fn prepare(
    input: ContextualSourceInput,
    bytes: &[u8],
    typography: AnalysisTypography,
    scale: f32,
) -> (System, PreparedText, CapturedScope, CapturedScope) {
    let (captured, analysis) = analyzed(input, typography);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let options = PreparationOptions::try_new(scale, PreparationOptions::default().tabs())
        .unwrap()
        .with_orientation(LineOrientation::Vertical);
    let prepared = system.prepare(&analysis, options).unwrap();
    (
        system,
        prepared,
        captured.captured_scope(0).unwrap(),
        captured.captured_scope(1).unwrap(),
    )
}
fn upright() -> AnalysisTypography {
    AnalysisTypography::Upright {
        computed_direction: ParagraphBaseDirection::RightToLeft,
    }
}
fn selected(prepared: &PreparedText) -> SelectedLine {
    let mut request = super::super::request(prepared);
    request.context.orientation = LineOrientation::Vertical;
    request.context.base_direction = ParagraphBaseDirection::LeftToRight;
    request.context.line_box_direction = LineBoxDirection::LeftToRight;
    request.context.available_inline = None;
    request.context.alignment = Alignment::Start;
    request.context.spacing = SpacingPolicy::None;
    prepared.select(request).unwrap()
}
fn finalized(selected: &SelectedLine, amount: f64) -> FinalizedLine {
    assert_eq!(selected.scopes().len(), 1);
    selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
                scope: selected.scopes().next().unwrap(),
                displacement: BaselineShift::try_new(amount).unwrap(),
            }]),
        })
        .unwrap()
}
fn place(prepared: &PreparedText, line: &FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap()
}
type Packet = (Range, GlyphBasis, u32, f32, f32, f32);
fn packets(line: &FinalizedLine) -> Vec<Packet> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        run.glyph_basis(),
                        g.id(),
                        g.x(),
                        g.y(),
                        g.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn central(metrics: PreparedScopeMetrics<'_>, coordinate: f64, script: [u8; 4]) {
    assert_eq!(metrics.orientation(), LineOrientation::Vertical);
    assert_eq!(
        metrics.common_baseline_reference(),
        Some(FontBaseline::Central)
    );
    assert_eq!(metrics.font_reference().axis, FontBaselineAxis::VerticalX);
    assert_eq!(metrics.native_reference_coordinate(), coordinate);
    assert_eq!(
        metrics.matching_coordinate(FontBaseline::Central),
        Some(0.0)
    );
    assert_eq!(metrics.matching_coordinate(FontBaseline::Alphabetic), None);
    assert_eq!(
        metrics.upright_reference(),
        Some(UprightMetricReference::FontDesignX)
    );
    assert_eq!(
        metrics.upright_provenance(),
        Some(UprightMetricProvenance::RawBaseline(
            FontBaselineProvenance::BaseEdgesCssMidpoint
        ))
    );
    let Some(FontBaselineOutcome::Available(value)) =
        metrics.baseline_outcome(FontBaseline::Central)
    else {
        panic!("real vertical BASE midpoint")
    };
    assert_eq!(value.coordinate(), coordinate);
    assert_eq!(value.base_script(), Some(script));
    assert_eq!(
        value.provenance(),
        FontBaselineProvenance::BaseEdgesCssMidpoint
    );
}

#[test]
fn native_vertical_parent_and_child_keep_actual_script_edges_scale_and_central_provenance() {
    let bytes = native_font();
    premises(&bytes);
    for scale in [1.0_f32, 2.0] {
        let (system, prepared, parent, child) =
            prepare(input("X", false), &bytes, upright(), scale);
        let p = prepared.scope_metrics(&parent).unwrap();
        let c = prepared.scope_metrics(&child).unwrap();
        let m = f64::from(scale);
        central(p, 16.0 * m, *b"hani");
        central(c, 6.0 * m, *b"latn");
        assert_eq!(p.font_reference().script.iso15924(), *b"Hani");
        assert_eq!(c.font_reference().script.iso15924(), *b"Latn");
        assert_eq!(
            (p.font_size(), p.line_height(), p.over(), p.under()),
            (20.0 * scale, 32.0 * m, 16.0 * m, 16.0 * m)
        );
        assert_eq!(
            (c.font_size(), c.line_height(), c.over(), c.under()),
            (10.0 * scale, 12.0 * m, 6.0 * m, 6.0 * m)
        );
        // The direct public observation qualifies the exact same retained
        // instance, not a glyph-derived or horizontal-metric proxy.
        let instance = p.effective_instance().unwrap();
        assert_eq!(instance.face().bytes(), bytes);
        let actual = instance
            .upright_metrics(
                prepared.font_catalogue_identity(),
                p.font_reference().script,
            )
            .unwrap();
        assert_eq!(
            (
                actual.em_under(),
                actual.em_over(),
                actual.central_coordinate_or_distance()
            ),
            (0.0, 32.0 * m, 16.0 * m)
        );
        assert_eq!(actual.base_script(), Some(*b"hani"));
        let selection = selected(&prepared);
        let target = selection.scopes().next().unwrap();
        assert_eq!(target.captured_scope(), child);
        assert_eq!(target.parent_metrics().unwrap().captured_scope(), parent);
        assert!(selection.unscoped_metric_contributions().next().is_none());
        let stats = system.stats();
        let line = finalized(&selection, 0.0);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (12.0 * m, 6.0 * m, 12.0 * m),
            "glyph-free parent32px receipt contributes no strut or second Central rebase"
        );
        assert_eq!(
            packets(&line),
            [(
                Range::new(0, 1),
                GlyphBasis::Unrotated,
                58,
                -6.0 * scale,
                9.0 * scale,
                12.0 * scale
            )]
        );
        assert_eq!(system.stats(), stats);
    }
}

#[test]
fn upright_scope_shift_moves_right_and_source_queries_use_each_real_cell_frame() {
    let bytes = native_font();
    let (system, prepared, _parent, child) = prepare(input("XX", true), &bytes, upright(), 1.0);
    let selection = selected(&prepared);
    let stats = system.stats();
    let zero = finalized(&selection, 0.0);
    let shifted = finalized(&selection, 3.0);
    let expected = [
        (Range::new(0, 1), GlyphBasis::Unrotated, 58, -6.0, 9.0, 12.0),
        (Range::new(1, 2), GlyphBasis::Unrotated, 58, -6.0, 9.0, 12.0),
    ];
    assert_eq!(packets(&zero), expected);
    assert_eq!(packets(&shifted), expected);
    assert_eq!(
        (
            shifted.feedback().advance,
            shifted.feedback().baseline,
            shifted.feedback().line_height
        ),
        (24.0, 9.0, 15.0)
    );
    assert_eq!(shifted.scope_allocations()[0].scope, child);
    assert_eq!(
        shifted.scope_allocations()[0].cumulative_displacement.get(),
        3.0
    );
    let old = place(&prepared, &zero);
    let placed = place(&prepared, &shifted);
    assert_eq!(old.bounds(), Rect::new(94.0, 20.0, 12.0, 24.0));
    assert_eq!(placed.bounds(), Rect::new(94.0, 20.0, 15.0, 24.0));
    assert_eq!(
        placed
            .run_views()
            .map(|run| (run.origin(), run.range()))
            .collect::<Vec<_>>(),
        [
            (Point::new(100.0, 20.0), Range::new(0, 1)),
            (Point::new(103.0, 32.0), Range::new(1, 2))
        ]
    );
    for (cursor, rect) in [
        (
            Cursor::new(0, Affinity::After),
            Rect::new(94.0, 20.0, 12.0, 1.0),
        ),
        (
            Cursor::new(1, Affinity::After),
            Rect::new(94.0, 32.0, 12.0, 1.0),
        ),
        (
            Cursor::new(2, Affinity::Before),
            Rect::new(97.0, 44.0, 12.0, 1.0),
        ),
    ] {
        assert_eq!(placed.cursor(cursor).unwrap().rect(), rect);
    }
    let scoped_start = placed
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(1, Affinity::After))
        .unwrap();
    assert_eq!(
        placed.position_geometry(&scoped_start).unwrap().rect(),
        Rect::new(97.0, 32.0, 12.0, 1.0)
    );
    let geometry = placed
        .selection(Selection::new(
            Cursor::new(0, Affinity::After),
            Cursor::new(2, Affinity::Before),
        ))
        .unwrap();
    assert_eq!(
        geometry
            .rects()
            .iter()
            .map(|r| r.rect())
            .collect::<Vec<_>>(),
        [
            Rect::new(94.0, 20.0, 12.0, 12.0),
            Rect::new(97.0, 32.0, 12.0, 12.0)
        ]
    );
    let Hit::Text(hit) = placed.hit(Point::new(103.0, 34.0)) else {
        panic!("actual shifted inside cell must hit text")
    };
    assert_eq!(hit.source_cursor(), Cursor::new(1, Affinity::After));
    let start = placed
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(0, Affinity::After))
        .unwrap();
    let end = placed
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(2, Affinity::Before))
        .unwrap();
    let source = placed.position_selection(&start, &end).unwrap();
    assert_eq!(source.source_text().code_units(), [88, 88]);
    assert_eq!(source.plaintext().code_units(), [88, 88]);
    let replay = shifted.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.scope_allocations(), shifted.scope_allocations());
    assert_eq!(packets(&replay), expected);
    assert_eq!(
        place(&prepared, &replay).bounds(),
        Rect::new(94.0, 20.0, 15.0, 24.0)
    );
    assert_eq!(old.bounds(), Rect::new(94.0, 20.0, 12.0, 24.0));
    assert_eq!(system.stats(), stats);
}

#[test]
fn unsupported_cross_axis_and_vertical_alphabetic_demands_refuse_without_changing_old_geometry() {
    let bytes = native_font();
    let (mut system, prepared, _parent, _child) =
        prepare(input("X", false), &bytes, upright(), 1.0);
    let line = finalized(&selected(&prepared), 3.0);
    let old = place(&prepared, &line);
    for (index, axis, name) in [
        (0, FontBaselineAxis::HorizontalY, FontBaseline::Central),
        (1, FontBaselineAxis::HorizontalY, FontBaseline::Central),
        (1, FontBaselineAxis::VerticalX, FontBaseline::Alphabetic),
    ] {
        let mut invalid = input("X", false);
        let slot = match &mut invalid.scopes[index] {
            SourceProcessingScope::Block { font_reference, .. }
            | SourceProcessingScope::Inline { font_reference, .. } => font_reference,
            _ => unreachable!(),
        };
        let value = slot.as_mut().unwrap();
        value.axis = axis;
        value.baseline = name;
        let (_, analysis) = analyzed(invalid, upright());
        let error = system
            .prepare(
                &analysis,
                PreparationOptions::default().with_orientation(LineOrientation::Vertical),
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
        assert!(
            matches!(error.detail(), Some(ErrorDetail::ScopeAlignment { reason: ScopeAlignmentError::UnsupportedReference { scope } }) if *scope == index)
        );
    }
    let stats = system.stats();
    assert_eq!(old.bounds(), Rect::new(97.0, 20.0, 12.0, 12.0));
    assert_eq!(
        place(
            &prepared,
            &line.with_spacing(SpacingAllocations::None).unwrap()
        )
        .bounds(),
        old.bounds()
    );
    assert_eq!(system.stats(), stats);
}

#[test]
fn genuine_absent_base_keeps_nominal_pen_upright_receipts_and_central_matching_zero() {
    let bytes = vertical::native();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert!(font.table_data(skrifa::Tag::new(b"BASE")).is_none());
    assert_eq!(font.vhea().unwrap().version().to_major_minor(), (1, 0));
    let mut value = input("X", false);
    let SourceProcessingScope::Inline {
        style: Some(style), ..
    } = &mut value.scopes[1]
    else {
        unreachable!()
    };
    style.line_height = LineHeight::Absolute(10.0);
    author_child_body(&mut value, Range::new(0, 1));
    let (system, prepared, parent, child) = prepare(value, &bytes, upright(), 1.0);
    for (scope, size, side) in [(&parent, 20.0, 10.0), (&child, 10.0, 5.0)] {
        let metrics = prepared.scope_metrics(scope).unwrap();
        assert_eq!(metrics.font_size(), size);
        assert_eq!((metrics.over(), metrics.under()), (side, side));
        assert_eq!(
            metrics.upright_reference(),
            Some(UprightMetricReference::NominalVerticalPenX)
        );
        assert_eq!(
            metrics.upright_provenance(),
            Some(UprightMetricProvenance::SynthesizedUprightEmAtNominalPen)
        );
        assert_eq!(metrics.native_reference_coordinate(), 0.0);
        assert_eq!(
            metrics.matching_coordinate(FontBaseline::Central),
            Some(0.0)
        );
        assert!(matches!(
            metrics.baseline_outcome(FontBaseline::Central),
            Some(FontBaselineOutcome::Unavailable(_))
        ));
    }
    let selection = selected(&prepared);
    let stats = system.stats();
    let line = finalized(&selection, 3.0);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (12.0, 8.0, 10.0)
    );
    assert_eq!(
        packets(&line),
        [(Range::new(0, 1), GlyphBasis::Unrotated, 58, -5.0, 9.0, 12.0)]
    );
    assert_eq!(
        place(&prepared, &line).bounds(),
        Rect::new(98.0, 20.0, 10.0, 12.0)
    );
    assert_eq!(system.stats(), stats);
}

#[test]
fn mixed_rotated_latin_and_native_han_share_the_scope_axis_without_changing_glyph_bases() {
    let bytes = crate::mixed_test_fonts::fixture(true);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('X'), Some(GlyphId::new(58)));
    assert_eq!(font.charmap().map('一'), Some(GlyphId::new(58)));
    assert_eq!(font.vmtx().unwrap().v_metrics()[82].advance.get(), 1200);
    assert_eq!(font.vmtx().unwrap().v_metrics()[82].side_bearing.get(), 100);
    // Existing owned GSUB explicitly maps vrtr58->274 and vert58->82. Their
    // actual Ahem outlines are respectively y400..600 and y-200..0.
    let raw = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for (id, y) in [(274, [400.0, 600.0]), (82, [-200.0, 0.0])] {
        let b = raw.bounds(GlyphId::new(id)).unwrap();
        assert_eq!([b.y_min, b.y_max], y);
    }
    let mut value = input("X一", true);
    let SourceProcessingScope::Block { font_reference, .. } = &mut value.scopes[0] else {
        unreachable!()
    };
    *font_reference = Some(reference(*b"Latn"));
    let SourceProcessingScope::Inline { font_reference, .. } = &mut value.scopes[1] else {
        unreachable!()
    };
    *font_reference = Some(reference(*b"Hani"));
    let (system, prepared, parent, child) = prepare(value, &bytes, AnalysisTypography::Mixed, 1.0);
    central(prepared.scope_metrics(&parent).unwrap(), 6.0, *b"latn");
    central(prepared.scope_metrics(&child).unwrap(), 6.0, *b"hani");
    let selection = selected(&prepared);
    assert_eq!(
        selection
            .unscoped_metric_contributions()
            .collect::<Vec<_>>(),
        [ScopeMetricContribution {
            before: 5.0,
            after: 7.0,
            alignment: LineBoxAlignment::Baseline
        }]
    );
    assert_eq!(
        selection
            .scopes()
            .next()
            .unwrap()
            .direct_metric_contributions()
            .collect::<Vec<_>>(),
        [ScopeMetricContribution {
            before: 6.0,
            after: 6.0,
            alignment: LineBoxAlignment::Baseline
        }]
    );
    let stats = system.stats();
    let line = finalized(&selection, 3.0);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (22.0, 9.0, 16.0)
    );
    assert_eq!(
        packets(&line),
        [
            (Range::new(0, 1), GlyphBasis::Clockwise, 274, 0.0, 0.0, 10.0),
            (Range::new(1, 2), GlyphBasis::Unrotated, 82, -6.0, 1.0, 12.0)
        ]
    );
    let placed = place(&prepared, &line);
    assert_eq!(placed.bounds(), Rect::new(93.0, 20.0, 16.0, 22.0));
    assert_eq!(
        placed
            .run_views()
            .map(|run| (run.range(), run.glyph_basis(), run.origin()))
            .collect::<Vec<_>>(),
        [
            (
                Range::new(0, 1),
                GlyphBasis::Clockwise,
                Point::new(96.0, 20.0)
            ),
            (
                Range::new(1, 2),
                GlyphBasis::Unrotated,
                Point::new(103.0, 30.0)
            )
        ]
    );
    assert_eq!(
        placed
            .cursor(Cursor::new(1, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(93.0, 30.0, 13.0, 1.0)
    );
    // Source cells retain the unallocated COMMON band6/7, not each font's
    // em interval. The actual Han reference moves +3 physical-right:103-7.
    let scoped_start = placed
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(1, Affinity::After))
        .unwrap();
    assert_eq!(
        placed.position_geometry(&scoped_start).unwrap().rect(),
        Rect::new(96.0, 30.0, 13.0, 1.0)
    );
    assert_eq!(placed.source().text().code_units(), [88, 19968]);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.scope_allocations(), line.scope_allocations());
    assert_eq!(place(&prepared, &replay).bounds(), placed.bounds());
    assert_eq!(system.stats(), stats);
}
