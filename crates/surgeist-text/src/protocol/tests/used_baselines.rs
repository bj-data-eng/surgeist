//! Explicit used references preserve raw font facts and shift the paint origin.
use super::*;
mod domains;
mod edges;
mod numeric;
mod placement_precision;
mod resources;
use crate::{
    Affinity, Cursor, FontBaseline, FontBaselineAxis, FontBaselineOutcome, FontBaselineProvenance,
    FontBaselineScript, GlyphBasis, Hit, Id, LineHeight, Point, Range, Rect, Selection,
    baseline_test_fonts as base, test_fonts as sfnt,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/ahem/Ahem.ttf"
));
const ALPHA: FontBaseline = FontBaseline::Alphabetic;
const AXIS: FontBaselineAxis = FontBaselineAxis::HorizontalY;

fn font(romn: i16) -> Vec<u8> {
    sfnt::replace_tables(
        AHEM,
        vec![(
            *b"BASE",
            base::base(0, Some(base::single_axis(&[*b"romn"], &[romn])), None, None),
        )],
    )
}

fn qualify(bytes: &[u8], romn: i16) {
    // Independently read the actual bytes before invoking Text preparation.
    for tag in [*b"hhea", *b"OS/2", *b"hmtx", *b"cmap", *b"glyf", *b"loca"] {
        assert_eq!(sfnt::table(bytes, tag), sfnt::table(AHEM, tag));
    }
    let parsed = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    let hhea = parsed.hhea().unwrap();
    assert_eq!(
        (
            hhea.ascender().to_i16(),
            hhea.descender().to_i16(),
            hhea.line_gap().to_i16()
        ),
        (800, -200, 0)
    );
    assert_eq!(sfnt::u16_at(sfnt::table(bytes, *b"OS/2"), 62) & 0x80, 0);
    assert!(parsed.table_data(skrifa::Tag::new(b"fvar")).is_none());
    assert_eq!(parsed.charmap().map('A'), Some(GlyphId::new(35)));
    let metrics = parsed.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(35)), Some(1000.0));
    let bounds = metrics.bounds(GlyphId::new(35)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
    let table = parsed.base().unwrap();
    assert_eq!((table.version().major, table.version().minor), (1, 0));
    assert!(table.vert_axis().is_none());
    let axis = table.horiz_axis().unwrap().unwrap();
    assert_eq!(
        axis.base_tag_list()
            .unwrap()
            .unwrap()
            .baseline_tags()
            .iter()
            .map(|tag| tag.get().to_be_bytes())
            .collect::<Vec<_>>(),
        [*b"romn"]
    );
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(scripts.base_script_records().len(), 1);
    let script = &scripts.base_script_records()[0];
    assert_eq!(script.base_script_tag(), skrifa::Tag::new(b"latn"));
    let values = script
        .base_script(scripts.offset_data())
        .unwrap()
        .base_values()
        .unwrap()
        .unwrap();
    assert_eq!(values.default_baseline_index(), 0);
    assert_eq!(
        values
            .base_coords()
            .iter()
            .map(|coord| coord.unwrap().coordinate())
            .collect::<Vec<_>>(),
        [romn]
    );
}

fn own_style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn analyzed(source: Source, style: Style) -> AnalyzedSource {
    let len = source.text().len();
    crate::test_wrapping::bind_missing(
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
                    source,
                    local: Range::new(0, len),
                    aggregate: Range::new(0, len),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: if len == 0 {
                    vec![crate::ParagraphAnchor {
                        item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                        affinity: Affinity::After,
                    }]
                } else {
                    vec![]
                },
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    )
}

fn finalize(prepared: &PreparedText) -> FinalizedLine {
    let mut selection = request(prepared);
    selection.context.orientation = prepared.orientation();
    selection.context.spacing = SpacingPolicy::None;
    // Line bounds retain the declared available span, separately from the
    // glyph advance and the selected original character's rectangle.
    selection.context.available_inline = Some(200.0);
    prepared
        .select(selection)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn placed(prepared: &PreparedText, line: FinalizedLine, baseline: f64) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline,
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn raw_reference(run: &crate::RunView<'_>, catalogue: &crate::FontCatalogueIdentity) {
    let instance = run.effective_instance().unwrap();
    assert_eq!(instance.font_size(), 10.0);
    assert_eq!(instance.axis_count(), 0);
    let metrics = instance.metrics(catalogue).unwrap();
    assert_eq!(
        (metrics.ascent(), metrics.descent(), metrics.leading()),
        (8.0, -2.0, 0.0)
    );
    let raw = run.baseline(catalogue, AXIS, ALPHA).unwrap();
    assert_eq!(
        raw.script(),
        FontBaselineScript::from_iso15924(*b"Latn").unwrap()
    );
    assert_eq!((raw.axis(), raw.name()), (AXIS, ALPHA));
    let FontBaselineOutcome::Available(value) = raw.outcome() else {
        panic!("actual romn must be available: {raw:?}")
    };
    assert_eq!(value.coordinate(), 1.0);
    assert_eq!(value.provenance(), FontBaselineProvenance::BaseCoordinate);
    assert_eq!(value.base_script(), Some(*b"latn"));
    assert_eq!(value.absence(), None);
}

#[test]
fn explicit_alphabetic_preserves_raw_metrics_and_places_font_zero_beyond_the_shared_reference() {
    // Adopted Inline3 A/D adjustment: raw A8/D-2 and b1 give used7/3,
    // H10. H common baseline70 puts font zero71; CW cross100 puts zero99.
    // This is new-front functional qualification, not a raw-Physical RED.
    let bytes = font(100);
    qualify(&bytes, 100);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), bytes);
    assert_eq!(face.index(), 0);
    let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
    assert_eq!(instance.axis_count(), 0);
    let catalogue = system.font_catalogue_identity().clone();
    let raw_metrics = instance.metrics(&catalogue).unwrap();
    assert_eq!((raw_metrics.ascent(), raw_metrics.descent()), (8.0, -2.0));
    let raw = instance
        .baseline(
            &catalogue,
            FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            AXIS,
            ALPHA,
        )
        .unwrap();
    assert!(
        matches!(raw.outcome(), FontBaselineOutcome::Available(value) if value.coordinate() == 1.0)
    );
    let source = Source::identified("A", Id::from_u64(1561001), 3);
    let input = analyzed(source.clone(), own_style());
    for (orientation, anchor, origin, bounds, first, last, basis) in [
        (
            LineOrientation::Horizontal,
            70.0,
            Point::new(20.0, 71.0),
            Rect::new(20.0, 63.0, 200.0, 10.0),
            Rect::new(20.0, 63.0, 1.0, 10.0),
            Rect::new(30.0, 63.0, 1.0, 10.0),
            GlyphBasis::Unrotated,
        ),
        (
            LineOrientation::SidewaysClockwise,
            100.0,
            Point::new(99.0, 20.0),
            Rect::new(97.0, 20.0, 10.0, 200.0),
            Rect::new(97.0, 20.0, 10.0, 1.0),
            Rect::new(97.0, 30.0, 10.0, 1.0),
            GlyphBasis::Clockwise,
        ),
    ] {
        let default = PreparationOptions::default().with_orientation(orientation);
        assert_eq!(default.requested_baseline_reference(), None);
        let control = system.prepare(&input, default).unwrap();
        assert_eq!(control.baseline_reference(), None);
        let raw_line = finalize(&control);
        assert_eq!(
            (
                raw_line.feedback().baseline,
                raw_line.feedback().line_height
            ),
            (8.0, 10.0)
        );
        let raw_layout = placed(&control, raw_line, anchor);
        raw_reference(&raw_layout.run_views().next().unwrap(), &catalogue);
        let options = default.with_baseline_reference(ALPHA);
        assert_eq!(options.requested_baseline_reference(), Some(ALPHA));
        assert_ne!(options, default);
        let prepared = system.prepare(&input, options).unwrap();
        assert_eq!(prepared.preparation_options(), options);
        assert_eq!(prepared.baseline_reference(), Some(ALPHA));
        assert_eq!(prepared.analyzed_source().identity(), input.identity());
        let line = finalize(&prepared);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (10.0, 7.0, 10.0)
        );
        assert_eq!(
            line.run_views().next().unwrap().origin(),
            Point::new(0.0, 8.0)
        );
        assert_eq!(
            line.run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|g| (g.id(), g.range(), g.x(), g.y(), g.pen(), g.advance()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [(35, Range::new(0, 1), 0.0, 0.0, 0.0, 10.0)]
        );
        let layout = placed(&prepared, line, anchor);
        assert_eq!(layout.source(), &source);
        assert_eq!(layout.bounds(), bounds);
        assert_eq!(layout.lines()[0].bounds(), bounds);
        assert_eq!(layout.lines()[0].baseline(), anchor);
        let run = layout.run_views().next().unwrap();
        assert_eq!(run.origin(), origin);
        assert_eq!(run.glyph_basis(), basis);
        assert_eq!(run.range(), Range::new(0, 1));
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.face().catalogue_identity(), &catalogue);
        raw_reference(&run, &catalogue);
        assert_eq!(layout.glyph_runs()[0].origin(), origin);
        let facts = layout.inline_metric_facts();
        assert_eq!(facts.lines()[0].line().baseline(), anchor);
        assert_eq!(
            facts.runs()[0].baseline(),
            if orientation == LineOrientation::Horizontal {
                71.0
            } else {
                99.0
            }
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::After))
                .unwrap()
                .rect(),
            first
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(1, Affinity::Before))
                .unwrap()
                .rect(),
            last
        );
        let selection = layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(1, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(selection.rects().len(), 1);
        assert_eq!(
            selection.rects()[0].rect(),
            if orientation == LineOrientation::Horizontal {
                Rect::new(20.0, 63.0, 10.0, 10.0)
            } else {
                Rect::new(97.0, 20.0, 10.0, 10.0)
            }
        );
        let point = if orientation == LineOrientation::Horizontal {
            Point::new(22.0, 65.0)
        } else {
            Point::new(99.0, 22.0)
        };
        let Hit::Text(position) = layout.hit(point) else {
            panic!("expected actual source hit")
        };
        assert_eq!(position.source_cursor(), Cursor::new(0, Affinity::After));
        assert_eq!(layout.position_geometry(&position).unwrap().rect(), first);
    }
}
