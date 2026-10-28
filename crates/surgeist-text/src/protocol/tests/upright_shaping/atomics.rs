//! Native cell allocations preserve an authenticated physical atomic box.
//! The cell is (inline,line-under); the child border/content stay physical.
use super::*;
use crate::{
    Affinity, ErrorDetail, FontBaselineScript, Id, InlineBox, InlineBoxKind, Layout,
    ParagraphAnchor, ParagraphAnchorItem, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    ParagraphOrigin, ParagraphOriginKind, Size as PhysicalSize, UprightMetricReference,
};

const BOX_ID: Id = Id::from_u64(1_552_001);
const STRUT: crate::LineStrut = crate::LineStrut {
    baseline: 8.0,
    line_extent: 10.0,
};

fn input() -> ParagraphInput {
    let mut source = Source::identified("XX", Id::from_u64(1_552_000), 1);
    source.inline_box(InlineBox::new(
        BOX_ID,
        InlineBoxKind::InFlow,
        1,
        PhysicalSize::new(6.0, 14.0),
    ));
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
        anchor_order: vec![ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(0),
            affinity: Affinity::After,
        }],
        boundary_policies: [
            ParagraphBoundaryDefinition::AtomicBefore(0),
            ParagraphBoundaryDefinition::AtomicAfter(0),
        ]
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

fn qualify_plain_ahem() {
    // Read the retained SFNT and its actual outline independently of shaping.
    // This font has no native vertical tables; no fabricated metric supplier
    // or borrowed BASE coordinate is needed to establish size4 em sides2/2.
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.maxp().unwrap().num_glyphs(), 281);
    for tag in [b"BASE", b"vhea", b"vmtx", b"fvar", b"CFF ", b"CFF2"] {
        assert!(font.table_data(skrifa::Tag::new(tag)).is_none());
    }
    assert!(font.glyf().is_ok());
    assert_eq!(font.charmap().map('X'), Some(GlyphId::new(58)));
    let hhea = sfnt::table(AHEM, *b"hhea");
    assert_eq!(
        [4, 6, 8].map(|at| i16::from_be_bytes(hhea[at..at + 2].try_into().unwrap())),
        [800, -200, 0]
    );
    let glyphs = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(glyphs.advance_width(GlyphId::new(58)), Some(1000.0));
    let bounds = glyphs.bounds(GlyphId::new(58)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
}

fn fixture() -> (System, AnalyzedSource) {
    qualify_plain_ahem();
    let mut system = system_with(AHEM);
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), AHEM);
    assert_eq!(face.index(), 0);
    let instance = face.unsynthesized_instance(4.0, &[]).unwrap();
    assert_eq!(instance.axis_count(), 0);
    let catalogue = system.font_catalogue_identity();
    let glyph = instance.vertical_glyph_metrics(58, catalogue).unwrap();
    assert_eq!(
        (glyph.advance(), glyph.origin_x(), glyph.origin_y()),
        (4.0, 2.0, 3.2)
    );
    let cross = instance
        .upright_metrics(
            catalogue,
            FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        )
        .unwrap();
    assert_eq!(
        cross.reference(),
        UprightMetricReference::NominalVerticalPenX
    );
    assert_eq!(
        (
            cross.central_coordinate_or_distance(),
            cross.used_over(),
            cross.used_under()
        ),
        (0.0, 2.0, -2.0)
    );

    let captured = input();
    assert_eq!(captured.source.text().code_units(), [88, 88]);
    assert_eq!(
        captured.source.boxes()[0].size(),
        PhysicalSize::new(6.0, 14.0)
    );
    let horizontal = system
        .prepare(
            &analyze(captured.clone(), AnalysisTypography::HorizontalComposition),
            PreparationOptions::default(),
        )
        .unwrap();
    let participants = horizontal.participants().collect::<Vec<_>>();
    let [
        PreparedParticipant::Text(before),
        PreparedParticipant::Atomic(atomic),
        PreparedParticipant::Text(after),
    ] = participants.as_slice()
    else {
        panic!("the actual mixed source order must be text, atomic, text");
    };
    assert_eq!(before.view().source_range(), Range::new(0, 1));
    assert_eq!(after.view().source_range(), Range::new(1, 2));
    let view = atomic.view();
    let original = view.original_box();
    assert_eq!(
        (
            original.id(),
            original.kind(),
            original.index(),
            original.size()
        ),
        (
            BOX_ID,
            InlineBoxKind::InFlow,
            1,
            PhysicalSize::new(6.0, 14.0)
        )
    );
    let typography = AnalysisTypography::Upright {
        computed_direction: ParagraphBaseDirection::RightToLeft,
    };
    let analysis = analyze(captured, typography);
    assert_eq!(analysis.typography(), typography);
    assert_eq!(analysis.direction(), Direction::LeftToRight);
    assert_eq!(
        analysis.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    (system, analysis)
}

fn allocation() -> AtomicLineAllocation {
    // Physical Central2 in a6-wide child is4 from its right line-over edge.
    // Native cell axes make physical6x14 a local14x6 allocation.
    AtomicLineAllocation::InFlow {
        flow: AtomicFlowAllocation {
            advance: 14.0,
            before_baseline: 4.0,
            after_baseline: 2.0,
            occupied_start: 0.0,
            occupied_end: 14.0,
        },
        box_geometry: AtomicBoxAllocation {
            border: Rect::new(0.0, -4.0, 14.0, 6.0),
            content: Rect::new(1.0, -3.0, 11.0, 4.0),
            baseline: 0.0,
        },
        alignment: LineBoxAlignment::Baseline,
    }
}

fn placement(
    prepared: &PreparedText,
    line: &FinalizedLine,
    atomics: Vec<AtomicPlacement>,
) -> PlacementRequest {
    PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics,
        })],
    }
}

#[test]
fn registered_nominal_font_and_authentic_atomic_source_qualify() {
    let _ = fixture();
}

#[test]
fn native_in_flow_atomic_keeps_physical_box_and_cross_baseline_through_public_placement() {
    let (mut system, analysis) = fixture();
    let prepared = system
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .expect("qualified in-flow atomic must prepare for native upright composition");
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(prepared.orientation(), LineOrientation::Vertical);
    assert_eq!(prepared.origin_views().count(), 1);
    assert_eq!(prepared.atomic_views().count(), 1);
    let atomic = prepared.atomic_views().next().unwrap().handle();
    assert_eq!(atomic.view().original_box().id(), BOX_ID);
    assert_eq!(atomic.view().source_position().get(), 1);
    let origin = prepared.origin_views().next().unwrap().handle();
    let mut request = request(&prepared);
    request.line_strut = Some(STRUT);
    request.context.orientation = LineOrientation::Vertical;
    request.context.available_inline = Some(22.0);
    request.context.alignment = crate::Alignment::Start;
    request.context.spacing = SpacingPolicy::None;
    request.atomics = vec![AtomicAllocation {
        occurrence: atomic.clone(),
        allocation: allocation(),
    }];
    let selected = prepared.select(request).unwrap();
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(line.prepared_text(), prepared);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (22.0, 8.0, 10.0)
    );
    let slots = line.atomic_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].occurrence, atomic);
    assert_eq!(slots[0].origin, Point::new(4.0, 8.0));
    assert_eq!(slots[0].allocation, allocation());
    let source_fragments = line
        .fragment_views()
        .map(|view| {
            assert_eq!(view.origin(), Some(origin.clone()));
            (view.aggregate_range(), view.local_range(), view.bounds())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        source_fragments,
        [
            (
                Some(Range::new(0, 1)),
                Some(Range::new(0, 1)),
                Some(Rect::new(0.0, 6.0, 4.0, 4.0))
            ),
            (
                Some(Range::new(1, 2)),
                Some(Range::new(1, 2)),
                Some(Rect::new(18.0, 6.0, 4.0, 4.0))
            ),
        ]
    );
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.glyph_basis(), crate::GlyphBasis::Unrotated);
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(
            run.face().catalogue_identity(),
            prepared.font_catalogue_identity()
        );
        glyphs.extend(run.glyphs().map(|glyph| {
            assert_eq!(
                (glyph.id(), glyph.x(), glyph.y(), glyph.advance()),
                (58, -2.0, 3.2, 4.0)
            );
            (glyph.range(), run.origin().x + glyph.pen())
        }));
    }
    assert_eq!(glyphs, [(Range::new(0, 1), 0.0), (Range::new(1, 2), 18.0)]);
    let actual = AtomicPlacement {
        occurrence: atomic.clone(),
        border: Rect::new(98.0, 24.0, 6.0, 14.0),
        content: Rect::new(99.0, 25.0, 4.0, 11.0),
        baseline: 100.0,
    };
    let layout = Layout::from_finalized_lines(placement(&prepared, &line, vec![actual.clone()]))
        .expect("the actual6x14 physical atomic must align on cross-axis CentralX100");
    assert_eq!(layout.source(), analysis.source());
    assert_eq!(layout.orientation(), LineOrientation::Vertical);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 2));
    assert_eq!(layout.lines()[0].baseline(), 100.0);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(98.0, 20.0, 10.0, 22.0)
    );
    let boxes = layout.inline_boxes();
    assert_eq!(boxes.len(), 1);
    assert_eq!(
        (
            boxes[0].id(),
            boxes[0].kind(),
            boxes[0].index(),
            boxes[0].line(),
            boxes[0].rect()
        ),
        (BOX_ID, InlineBoxKind::InFlow, 1, 0, actual.border)
    );
    assert_eq!(
        layout.hit(Point::new(100.0, 30.0)),
        crate::Hit::InlineBox(BOX_ID)
    );
    for (y, index) in [(20.5, 0), (38.5, 1)] {
        let crate::Hit::Text(hit) = layout.hit(Point::new(100.0, y)) else {
            panic!("real source cell must retain its text hit")
        };
        assert_eq!(
            hit.source_cursor(),
            crate::Cursor::new(index, Affinity::After)
        );
    }
    let placed_glyphs = layout
        .glyph_runs()
        .into_iter()
        .flat_map(|run| {
            assert_eq!(run.glyph_basis(), crate::GlyphBasis::Unrotated);
            run.glyphs()
                .iter()
                .map(|glyph| {
                    (
                        glyph.range(),
                        Point::new(
                            run.origin().x + f64::from(glyph.x()),
                            run.origin().y + glyph.pen() + f64::from(glyph.y()),
                        ),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        placed_glyphs,
        [
            (
                Range::new(0, 1),
                Point::new(98.0, 20.0 + f64::from(3.2_f32))
            ),
            (
                Range::new(1, 2),
                Point::new(98.0, 38.0 + f64::from(3.2_f32))
            ),
        ]
    );
    // Keep each custody/geometry check live. In particular the old physical-y
    // baseline24 must not be accepted as Vertical's cross-axis baseline100.
    for wrong in [
        AtomicPlacement {
            baseline: 24.0,
            ..actual.clone()
        },
        AtomicPlacement {
            border: Rect::new(97.0, 24.0, 6.0, 14.0),
            ..actual.clone()
        },
        AtomicPlacement {
            content: Rect::new(99.0, 26.0, 4.0, 11.0),
            ..actual.clone()
        },
    ] {
        let error =
            Layout::from_finalized_lines(placement(&prepared, &line, vec![wrong])).unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::AtomicPlacementMismatch {
                    line: 0,
                    occurrence: 0
                },
            })
        );
    }
    let missing = Layout::from_finalized_lines(placement(&prepared, &line, vec![])).unwrap_err();
    assert_eq!(
        missing.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::MissingAtomic { occurrence: 0 },
        })
    );
    let duplicate = Layout::from_finalized_lines(placement(
        &prepared,
        &line,
        vec![actual.clone(), actual.clone()],
    ))
    .unwrap_err();
    assert_eq!(
        duplicate.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::DuplicateAtomic { occurrence: 0 },
        })
    );
    drop(system);
    assert_eq!(line.atomic_slots().next().unwrap().occurrence, atomic);
    assert_eq!(
        Layout::from_finalized_lines(placement(&prepared, &line, vec![actual]))
            .unwrap()
            .bounds(),
        layout.bounds()
    );
}
