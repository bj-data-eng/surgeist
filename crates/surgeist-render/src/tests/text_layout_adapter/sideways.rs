//! The direct adapter must execute the actual CW basis on retained outlines.
//! Ahem capital chi274 is a horizontal stripe, not a rotation-invariant square.
use super::*;
use kurbo::Shape as _;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
};

fn clockwise(mut authored: text::Style) -> (text::Layout, text::System) {
    authored.size = 10.0;
    authored.line_height = text::LineHeight::Absolute(10.0);
    let mut s = system(AHEM.to_vec());
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.charmap().map(0x3a7_u32).unwrap().to_u32(), 274);
    let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(274)), Some(1000.0));
    let bounds = metrics.bounds(GlyphId::new(274)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, 400.0, 1000.0, 600.0]
    );
    let a = text::AnalyzedSource::new(
        text::Source::identified("\u{3a7}", text::Id::from_u64(1515005), 5),
        authored,
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let p = s
        .prepare(
            &a,
            text::PreparationOptions::default()
                .with_orientation(text::LineOrientation::SidewaysClockwise),
        )
        .unwrap();
    assert_eq!(p.orientation(), text::LineOrientation::SidewaysClockwise);
    let end = p
        .opportunity_views()
        .find(|o| o.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = p
        .select(text::LineSelectionRequest {
            start: p.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: text::PreparedTermination::Ordinary(end),
            participants: p.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: text::LineOrientation::SidewaysClockwise,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: text::Alignment::Left,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(text::SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(
        line.run_views()
            .flat_map(|r| r
                .glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(274, 10.0)]
    );
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: p,
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: text::LineOrientation::SidewaysClockwise,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        text::Point::new(100.0, 20.0)
    );
    assert_eq!(
        layout.run_views().next().unwrap().orientation(),
        text::LineOrientation::SidewaysClockwise
    );
    (layout, s)
}

fn commanded_ink(run: &PreparedGlyphRun, transform: Transform) -> kurbo::Rect {
    // Execute the actual scene command on the actual retained vector path,
    // rather than accepting correct metadata alongside horizontal painting.
    let mut path = run.path().clone();
    path.apply_affine(kurbo::Affine::new(transform.as_array()));
    path.bounding_box()
}

#[test]
fn clockwise_asymmetric_outline_uses_one_basis_then_real_outer_placement() {
    let (input, s) = clockwise(style());
    let stats = s.stats();
    assert_eq!(input.source().text().code_units(), [0x3a7]);
    assert_eq!(input.source().id(), Some(text::Id::from_u64(1515005)));
    assert_eq!(input.source().revision(), 5);
    for (outer, coefficients, ink) in [
        (
            Transform::identity(),
            [0.0, 1.0, -1.0, 0.0, 100.0, 20.0],
            kurbo::Rect::new(104.0, 20.0, 106.0, 30.0),
        ),
        (
            Transform::translation(7.0, 9.0).unwrap(),
            [0.0, 1.0, -1.0, 0.0, 107.0, 29.0],
            kurbo::Rect::new(111.0, 29.0, 113.0, 39.0),
        ),
        (
            Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
            [0.0, 3.0, -2.0, 0.0, 207.0, 69.0],
            kurbo::Rect::new(215.0, 69.0, 219.0, 99.0),
        ),
    ] {
        let prepared = prepare_text_layout(&input, outer, limits()).unwrap();
        assert!(std::ptr::eq(prepared.layout(), &input));
        assert_eq!(s.stats(), stats);
        assert_eq!(prepared.scene().commands.len(), 1);
        let (run, transform) = outline(&prepared.scene().commands[0]);
        assert_eq!(transform.as_array(), coefficients);
        assert_eq!(run.font().bytes(), AHEM);
        assert_eq!(run.font().index(), 0);
        assert_eq!(run.instance().size(), 10.0);
        assert!(run.instance().coordinates().is_empty());
        assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
        assert_eq!(
            run.glyphs(),
            [TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap()]
        );
        // Real unhinted chi: x0..1000,y400..600 at UPEM1000; y reflected.
        assert_eq!(
            run.bounds().ink_rect(),
            Some(Rect::try_new(0.0, -6.0, 10.0, 2.0).unwrap())
        );
        assert_eq!(commanded_ink(run, transform), ink);
    }
    let failure = refuse(
        &input,
        Transform::identity(),
        bounded(0, 1024, ROOM, 32, 32, 32),
    );
    assert_limit(&failure, TextLayoutResource::Glyphs, 1, 0);
    assert_eq!(input.source().text().code_units(), [0x3a7]);
    assert_eq!(
        input.run_views().next().unwrap().origin(),
        text::Point::new(100.0, 20.0)
    );
}

#[test]
fn clockwise_italic_executes_horizontal_shear_before_rotation_and_outer_scale() {
    let authored = text::Style {
        font: text::Font::new().family("Ahem").style(text::Slant::Italic),
        ..style()
    };
    let (input, s) = clockwise(authored);
    let text_run = input.run_views().next().unwrap();
    let instance = text_run.effective_instance().unwrap();
    assert!(instance.x_shear() > 0.0);
    assert_eq!(instance.embolden(), 0.0);
    let outer = Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap();
    let before = s.stats();
    let prepared = prepare_text_layout(&input, outer, limits()).unwrap();
    assert_eq!(s.stats(), before);
    let (run, transform) = outline(&prepared.scene().commands[0]);
    assert_eq!(run.font().bytes(), instance.face().bytes());
    assert_eq!(run.font().index(), instance.face().index());
    assert_eq!(
        run.instance().synthesis(),
        GlyphSynthesis::try_new(instance.x_shear(), 0.0).unwrap()
    );
    assert_eq!(transform.as_array(), [0.0, 3.0, -2.0, 0.0, 207.0, 69.0]);
    assert_eq!(
        run.glyphs(),
        [TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap()]
    );
    // Font-up y4..6 shears x by4s..6s before y reflection. The retained
    // outline's f32 vertices are then transformed once by the scene command.
    let left = f64::from((4.0 * f64::from(instance.x_shear())) as f32);
    let right = f64::from((10.0 + 6.0 * f64::from(instance.x_shear())) as f32);
    assert_eq!(
        run.bounds().ink_rect(),
        Some(Rect::try_new(left, -6.0, right - left, 2.0).unwrap())
    );
    assert_eq!(
        commanded_ink(run, transform),
        kurbo::Rect::new(215.0, 69.0 + 3.0 * left, 219.0, 69.0 + 3.0 * right)
    );
}

#[test]
fn clockwise_decoration_enters_scene_as_already_physical_geometry() {
    let authored = text::Style {
        underline: text::Decoration::solid()
            .with_offset(text::DecorationOffset::try_absolute(-2.0).unwrap())
            .with_thickness(text::DecorationThickness::try_absolute(1.0).unwrap()),
        ..style()
    };
    let (input, _s) = clockwise(authored);
    assert_eq!(
        input.decoration_views().next().unwrap().rect(),
        text::Rect::new(97.0, 20.0, 1.0, 10.0)
    );
    let prepared = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    assert_eq!(prepared.scene().commands.len(), 2);
    let Command::Fill { shape, .. } = &prepared.scene().commands[1] else {
        panic!("actual physical decoration fill required")
    };
    assert_eq!(
        *shape,
        crate::Shape::rect(Rect::try_new(97.0, 20.0, 1.0, 10.0).unwrap())
    );
    let (run, transform) = outline(&prepared.scene().commands[0]);
    assert_eq!(
        commanded_ink(run, transform),
        kurbo::Rect::new(104.0, 20.0, 106.0, 30.0)
    );
}
