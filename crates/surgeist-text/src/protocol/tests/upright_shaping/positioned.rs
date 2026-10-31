//! An actual positioned occurrence is retained without becoming line content.
use super::*;
#[path = "positioned/supplemental.rs"]
mod supplemental;
use crate::{
    Affinity, FontBaselineScript, Id, InlineBox, InlineBoxKind, Layout, ParagraphAnchor,
    ParagraphAnchorItem, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphOrigin,
    ParagraphOriginKind, Size as PhysicalSize,
};

const BOX: Id = Id::from_u64(1_554_001);
const STRUT: crate::LineStrut = crate::LineStrut {
    baseline: 8.0,
    line_extent: 10.0,
};

fn input(positioned: bool) -> ParagraphInput {
    let mut source = Source::identified("XX", Id::from_u64(1_554_000), 1);
    if positioned {
        source.inline_box(InlineBox::new(
            BOX,
            InlineBoxKind::OutOfFlow,
            1,
            PhysicalSize::new(60.0, 140.0),
        ));
    }
    ParagraphInput {
        source: source.clone(),
        base_style: Style {
            font: Font::new().family("Ahem"),
            size: 4.0,
            line_height: LineHeight::Absolute(4.0),
            white_space: WhiteSpace::Preserve,
            wrap: crate::Wrap::Word,
            ..Style::default()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, 2),
            aggregate: Range::new(0, 2),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        replacements: vec![],
        anchor_order: if positioned {
            vec![ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(0),
                affinity: Affinity::After,
            }]
        } else {
            vec![]
        },
        boundary_policies: if positioned {
            vec![
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
        } else {
            vec![]
        }
        .into_iter()
        .map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect(),
    }
}

fn analyze(input: ParagraphInput, typography: AnalysisTypography) -> AnalyzedSource {
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, typography).unwrap(),
        crate::Wrap::Word,
    )
}

fn finish(prepared: &PreparedText) -> FinalizedLine {
    let mut request = request(prepared);
    request.context.orientation = LineOrientation::Vertical;
    request.context.available_inline = Some(8.0);
    request.context.alignment = crate::Alignment::Start;
    request.context.spacing = SpacingPolicy::None;
    request.line_strut = Some(STRUT);
    request.atomics = prepared
        .atomic_views()
        .map(|view| AtomicAllocation {
            occurrence: view.handle(),
            allocation: AtomicLineAllocation::OutOfFlow,
        })
        .collect();
    prepared
        .select(request)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

#[test]
fn native_positioned_atomic_keeps_independent_physical_geometry_without_flow_contribution() {
    // Positive font and producer premises precede the expected admission success.
    let parsed = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(parsed.charmap().map('X'), Some(GlyphId::new(58)));
    for tag in [b"BASE", b"vhea", b"vmtx", b"fvar"] {
        assert!(parsed.table_data(skrifa::Tag::new(tag)).is_none());
    }
    let glyphs = parsed.glyph_metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(glyphs.advance_width(GlyphId::new(58)), Some(1000.0));
    let mut system = system_with(AHEM);
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), AHEM);
    assert_eq!(face.index(), 0);
    let instance = face.unsynthesized_instance(4.0, &[]).unwrap();
    assert_eq!(instance.axis_count(), 0);
    let catalogue = system.font_catalogue_identity();
    let nominal = instance.vertical_glyph_metrics(58, catalogue).unwrap();
    assert_eq!(
        (nominal.advance(), nominal.origin_x(), nominal.origin_y()),
        (4.0, 2.0, 3.2)
    );
    let cross = instance
        .upright_metrics(
            catalogue,
            FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        )
        .unwrap();
    assert_eq!((cross.used_over(), cross.used_under()), (2.0, -2.0));
    let typography = AnalysisTypography::Upright {
        computed_direction: ParagraphBaseDirection::RightToLeft,
    };
    let control = system
        .prepare(
            &analyze(input(false), typography),
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let plain = finish(&control);
    assert_eq!(
        (
            plain.feedback().advance,
            plain.feedback().baseline,
            plain.feedback().line_height
        ),
        (8.0, 8.0, 10.0)
    );
    assert_eq!(
        control
            .opportunity_views()
            .filter(|o| o.kind() == PreparedBreakKind::Allowed)
            .count(),
        0
    );

    let captured = input(true);
    let horizontal = system
        .prepare(
            &analyze(captured.clone(), AnalysisTypography::HorizontalComposition),
            PreparationOptions::default(),
        )
        .unwrap();
    let actual = horizontal.atomic_views().next().unwrap();
    assert_eq!(
        (
            actual.original_box().id(),
            actual.original_box().kind(),
            actual.source_position().get()
        ),
        (BOX, InlineBoxKind::OutOfFlow, 1)
    );
    let source = captured.source.clone();
    let analysis = analyze(captured, typography);
    assert_eq!(analysis.direction(), Direction::LeftToRight);
    assert_eq!(analysis.typography(), typography);
    let prepared = system.prepare(&analysis, PreparationOptions::default().with_orientation(LineOrientation::Vertical))
        .expect("qualified native upright positioned occurrence must prepare without a flow placeholder");
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(
        prepared
            .opportunity_views()
            .filter(|o| o.kind() == PreparedBreakKind::Allowed)
            .count(),
        0
    );
    let line = finish(&prepared);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (8.0, 8.0, 10.0)
    );
    let atomic = prepared.atomic_views().next().unwrap().handle();
    let slots = line.atomic_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].occurrence, atomic);
    assert_eq!(slots[0].allocation, AtomicLineAllocation::OutOfFlow);
    assert_eq!(slots[0].origin, Point::new(4.0, 8.0));
    let origin = prepared.origin_views().next().unwrap().handle();
    let fragments = line
        .fragment_views()
        .map(|view| {
            assert_eq!(view.origin(), Some(origin.clone()));
            (view.aggregate_range(), view.local_range(), view.bounds())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        fragments,
        [
            (
                Some(Range::new(0, 1)),
                Some(Range::new(0, 1)),
                Some(Rect::new(0.0, 6.0, 4.0, 4.0))
            ),
            (
                Some(Range::new(1, 2)),
                Some(Range::new(1, 2)),
                Some(Rect::new(4.0, 6.0, 4.0, 4.0))
            ),
        ]
    );
    // This actual rectangle and finite baseline belong to the positioned box;
    // neither is constrained to the static flow slot or the line's baseline.
    let physical = AtomicPlacement {
        occurrence: atomic,
        border: Rect::new(300.0, 500.0, 60.0, 140.0),
        content: Rect::new(305.0, 507.0, 48.0, 126.0),
        baseline: 321.0,
    };
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![physical.clone()],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), &source);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].bounds(), Rect::new(98.0, 20.0, 10.0, 8.0));
    assert_eq!(layout.inline_boxes()[0].rect(), physical.border);
    assert_eq!(layout.inline_boxes()[0].id(), BOX);
    assert_eq!(layout.inline_boxes()[0].kind(), InlineBoxKind::OutOfFlow);
    assert_eq!(
        layout.hit(Point::new(310.0, 510.0)),
        crate::Hit::InlineBox(BOX)
    );
    let glyphs = layout
        .glyph_runs()
        .into_iter()
        .flat_map(|run| {
            assert_eq!(run.glyph_basis(), crate::GlyphBasis::Unrotated);
            run.glyphs()
                .iter()
                .map(|g| {
                    assert_eq!((g.id(), g.x(), g.y(), g.advance()), (58, -2.0, 3.2, 4.0));
                    (g.range(), run.origin().x, run.origin().y + g.pen())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [
            (Range::new(0, 1), 100.0, 20.0),
            (Range::new(1, 2), 100.0, 24.0)
        ]
    );
    drop(system);
    assert_eq!(
        line.atomic_slots().next().unwrap().allocation,
        AtomicLineAllocation::OutOfFlow
    );
}
