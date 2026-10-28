//! Whole-upright placement through the existing native paragraph front.
//!
//! Pinned Writing Modes 4 §§4.2, 5.1 and 5.1.1 require Central, used LTR,
//! unrotated Latin outlines and vertical metrics. The adopted native upright
//! baseline and vertical-glyph determinations define the font-coordinate policy.
use super::*;
use crate::{
    AnalysisTypography, Direction, Id, LineBoxDirection, LineHeight, Point, Range, Rect, Size,
    baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

fn native_font() -> Vec<u8> {
    let horizontal = vertical::horizontal(800, -200, 0, 1001);
    let count = sfnt::u16_at(sfnt::table(&horizontal, *b"maxp"), 4);
    let native = vertical::with_native(&horizontal, 1200, 100, count);
    sfnt::replace_tables(
        &native,
        vec![(
            *b"BASE",
            base::base(
                0,
                None,
                Some(base::single_axis(&[*b"ideo", *b"idtp"], &[0, 1200])),
                None,
            ),
        )],
    )
}

fn qualify_font(bytes: &[u8]) {
    // Read actual font tables and outlines independently of Text's metrics and
    // shaping output. In particular W = 1001 distinguishes nominal Ox = 500.5
    // from the absolute BASE Central = 600 used by upright positioning.
    let face = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.maxp().unwrap().num_glyphs(), 281);
    for tag in [b"fvar", b"CFF ", b"CFF2"] {
        assert!(face.table_data(skrifa::Tag::new(tag)).is_none());
    }
    assert!(face.glyf().is_ok());
    assert_eq!(face.charmap().map('X').unwrap().to_u32(), 58);
    let glyphs = face.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    assert_eq!(glyphs.advance_width(GlyphId::new(58)), Some(1001.0));
    let bounds = glyphs.bounds(GlyphId::new(58)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
    let vhea = face.vhea().unwrap();
    assert_eq!(vhea.version().to_major_minor(), (1, 0));
    assert_eq!(vhea.number_of_long_ver_metrics(), 281);
    let metrics = face.vmtx().unwrap();
    let x = &metrics.v_metrics()[58];
    assert_eq!((x.advance.get(), x.side_bearing.get()), (1200, 100));
    let base = face.base().unwrap();
    assert_eq!((base.version().major, base.version().minor), (1, 0));
    assert!(base.horiz_axis().is_none());
    let axis = base.vert_axis().unwrap().unwrap();
    let tags = axis.base_tag_list().unwrap().unwrap();
    assert_eq!(
        tags.baseline_tags()
            .iter()
            .map(|tag| tag.get().to_be_bytes())
            .collect::<Vec<_>>(),
        [*b"ideo", *b"idtp"]
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
    assert_eq!(values.base_coord_count(), 2);
    assert_eq!(values.default_baseline_index(), 0);
    assert_eq!(
        values
            .base_coords()
            .iter()
            .map(|coord| {
                let coord = coord.unwrap();
                (coord.base_coord_format(), coord.coordinate())
            })
            .collect::<Vec<_>>(),
        [(1, 0), (1, 1200)]
    );
}

#[test]
fn qualified_native_upright_reaches_placed_geometry() {
    let source = Source::identified("X", Id::from_u64(7811533), 1);
    let typography = AnalysisTypography::Upright {
        computed_direction: ParagraphBaseDirection::RightToLeft,
    };
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    };
    let analysis = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: style.clone(),
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(0, 1),
                    aggregate: Range::new(0, 1),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            typography,
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analysis.typography(), typography);
    assert_eq!(analysis.direction(), Direction::LeftToRight);
    assert_eq!(analysis.source(), &source);
    assert_eq!(analysis.source().text().code_units(), [88]);
    assert_eq!(analysis.graphemes().collect::<Vec<_>>(), [Range::new(0, 1)]);
    assert_eq!(analysis.style(), &style);

    let bytes = native_font();
    qualify_font(&bytes);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let faces = system.registered_font_faces().collect::<Vec<_>>();
    assert_eq!(faces.len(), 1);
    let face = faces[0].to_owned();
    assert_eq!(face.bytes(), bytes);
    assert_eq!(face.index(), 0);
    assert_eq!(
        face.unsynthesized_instance(10.0, &[]).unwrap().axis_count(),
        0
    );
    let catalogue = system.font_catalogue_identity().clone();
    let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
    let nominal = instance.vertical_glyph_metrics(58, &catalogue).unwrap();
    assert_eq!(
        (nominal.advance(), nominal.origin_x(), nominal.origin_y()),
        (12.0, 5.005, 9.0)
    );
    let cross = instance
        .upright_metrics(
            &catalogue,
            crate::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        )
        .unwrap();
    assert_eq!(
        cross.reference(),
        crate::UprightMetricReference::FontDesignX
    );
    assert_eq!(
        (
            cross.central_coordinate_or_distance(),
            cross.used_under(),
            cross.used_over()
        ),
        (6.0, -6.0, 6.0)
    );
    let prepared = system
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .expect("qualified whole-upright analysis must prepare with native vertical metrics");
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(prepared.orientation(), LineOrientation::Vertical);
    assert_eq!(prepared.font_catalogue_identity(), &catalogue);
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| (
                unit.source_range(),
                unit.advance(),
                unit.ascent(),
                unit.descent()
            ))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 12.0, 6.0, 6.0)]
    );
    let mut selected_request = request(&prepared);
    selected_request.context.orientation = LineOrientation::Vertical;
    selected_request.context.base_direction = ParagraphBaseDirection::LeftToRight;
    selected_request.context.line_box_direction = LineBoxDirection::LeftToRight;
    selected_request.context.available_inline = None;
    selected_request.context.indent = 0.0;
    selected_request.context.alignment = crate::Alignment::Start;
    selected_request.context.spacing = SpacingPolicy::None;
    let line = prepared
        .select(selected_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    // Scale design units once by 10/1000. Central = (0+1200)/2 = 600;
    // em sides 600/600, advance 1200, effective origin (600,800+100).
    // Unrotated outline offset is (-600,+900), independently of W/2.
    assert_eq!(line.feedback().advance, 12.0);
    assert_eq!(line.feedback().line_height, 12.0);
    assert_eq!(
        (line.feedback().ascent, line.feedback().descent),
        (6.0, 6.0)
    );
    let finalized = line.run_views().collect::<Vec<_>>();
    assert_eq!(finalized.len(), 1);
    assert_eq!(
        finalized[0]
            .glyphs()
            .map(|g| (g.id(), g.x(), g.y(), g.advance(), g.pen(), g.range()))
            .collect::<Vec<_>>(),
        [(58, -6.0, 9.0, 12.0, 0.0, Range::new(0, 1))]
    );
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    drop(system);
    assert_eq!(layout.source(), &source);
    assert_eq!(layout.orientation(), LineOrientation::Vertical);
    assert_eq!(layout.lines().len(), 1);
    // Cell geometry maps inline [0,12], cross [-6,6] about cross 100/inline 20.
    let bounds = Rect::new(94.0, 20.0, 12.0, 12.0);
    assert_eq!(layout.lines()[0].bounds(), bounds);
    assert_eq!(layout.bounds(), bounds);
    assert_eq!(layout.metrics().size(), Size::new(12.0, 12.0));
    let runs = layout.run_views().collect::<Vec<_>>();
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(run.orientation(), LineOrientation::Vertical);
    assert_eq!(run.origin(), Point::new(100.0, 20.0));
    assert_eq!(run.range(), Range::new(0, 1));
    assert_eq!(run.face().bytes(), bytes);
    assert_eq!(run.face().index(), 0);
    assert_eq!(run.face().catalogue_identity(), &catalogue);
    assert!(run.face().is_same_face(&face.face()));
    let instance = run.effective_instance().unwrap();
    assert_eq!(instance.font_size(), 10.0);
    assert_eq!(instance.normalized_coords().collect::<Vec<_>>(), []);
    let glyphs = run.glyphs().collect::<Vec<_>>();
    assert_eq!(glyphs.len(), 1);
    let glyph = glyphs[0];
    assert_eq!(
        (
            glyph.id(),
            glyph.x(),
            glyph.y(),
            glyph.advance(),
            glyph.pen()
        ),
        (58, -6.0, 9.0, 12.0, 0.0)
    );
    // Upright glyph axes stay unrotated: offset x changes physical x, while
    // the inline pen and offset y change physical y. Cell rotation is separate.
    assert_eq!(
        Point::new(
            run.origin().x + f64::from(glyph.x()),
            run.origin().y + glyph.pen() + f64::from(glyph.y())
        ),
        Point::new(94.0, 29.0)
    );
    let owned = layout.glyph_runs();
    assert_eq!(owned.len(), 1);
    assert_eq!(owned[0].origin(), run.origin());
    assert_eq!(owned[0].glyphs().len(), 1);
    let owned_glyph = owned[0].glyphs()[0];
    assert_eq!(owned_glyph.range(), Range::new(0, 1));
    assert_eq!(
        (
            owned_glyph.id(),
            owned_glyph.x(),
            owned_glyph.y(),
            owned_glyph.advance(),
            owned_glyph.pen()
        ),
        (58, -6.0, 9.0, 12.0, 0.0)
    );
}
